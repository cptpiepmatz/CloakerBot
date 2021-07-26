import {GuildMember} from "discord.js";
import {readdirSync} from "fs";
import {join} from "path";
import {
  AudioPlayerStatus,
  AudioResource,
  createAudioPlayer,
  createAudioResource,
  joinVoiceChannel,
  StreamType
} from "@discordjs/voice";
import AwaitLock from "await-lock";
import sleep from "sleep-promise";
import pEvent from "p-event";
import {Readable} from "stream";

import getRandom from "./util/getRandom";
import audioBuffers from "../provider/audioBuffers";

const takedownFiles =
  readdirSync(join(__dirname, "../assets/sounds/generated/takedowns"));
const failFiles =
  readdirSync(join(__dirname, "../assets/sounds/generated/fails"));

/**
 * Creates an random audio stream for the takedown and fail sounds.
 * @param kind For which kind of post message the resource shall be created
 */
function createRandomAudioResource(
  kind: "takedowns" | "fails"
): AudioResource<unknown> {
  let buffer = getRandom(Object.values(audioBuffers["dir:" + kind]));
  return createAudioResource(Readable.from(buffer), {
    inputType: StreamType.OggOpus
  });
}

async function takedown(member: GuildMember): Promise<boolean> {
  let takedownSuccessful = false;
  const guildId = member.guild.id;
  const takedownResource = createRandomAudioResource("takedowns");
  const failResource = createRandomAudioResource("fails");
  const wululuResource = createAudioResource(
    Readable.from(audioBuffers.WULULULULU as Buffer),
    {inputType: StreamType.OggOpus}
  );

  if (!takedownLocks[guildId]) takedownLocks[guildId] = new AwaitLock();
  const lock = takedownLocks[guildId];
  await lock.acquireAsync();

  if (!member.voice.channel) throw new Error("lol, kein Voice");

  const connection = joinVoiceChannel({
    channelId: member.voice.channel.id,
    guildId: guildId,
    adapterCreator: member.guild.voiceAdapterCreator
  });

  const player = createAudioPlayer();
  connection.subscribe(player);
  player.play(wululuResource);

  // Calculate time of the wululu
  // min   60000
  // max  230000
  // diff 170000
  const wululuTime = 1000 + Math.floor(Math.random() * 4000);
  await sleep(wululuTime);

  await member.voice.kick();
  takedownSuccessful = true;
  player.play(takedownResource);

  await pEvent(player, AudioPlayerStatus.Idle);
  player.stop();
  connection.destroy();
  lock.release();
  return takedownSuccessful;
}

export default takedown;
