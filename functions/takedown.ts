import {GuildMember} from "discord.js";
import {
  AudioPlayer,
  AudioPlayerStatus,
  AudioResource,
  createAudioPlayer,
  createAudioResource,
  joinVoiceChannel,
  StreamType,
  VoiceConnection
} from "@discordjs/voice";
import AwaitLock from "await-lock";
import sleep from "sleep-promise";
import pEvent from "p-event";
import {Readable} from "stream";

import getRandom from "./util/getRandom";
import audioBuffers from "../provider/audioBuffers";
import TakedownResult from "../types/TakedownResult";
import replies from "../provider/replies";
import Stats from "../enums/Stats";
import incStat from "./incStat";
import Logger from "js-logger";
import memberString from "./util/memberString";
import guildString from "./util/guildString";

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

/**
 * Executes the actual takedown.
 * Joins a voice channel of the given member, wulululus, kicks the member,
 * chooses a voice line and leaves.
 *
 * Currently making use of pre converted .ogg files and buffered pre buffered
 * audio to increase performance drastically.
 * @param member Member to take down
 * @param commander If given, this will increase the commander stat
 * @return A boolean presenting if the takedown did work
 */
async function takedown(
  member: GuildMember,
  commander: GuildMember | null = null
): Promise<TakedownResult> {
  Logger.info("initiating takedown " + memberString(member) +
    " on " + member.guild.name + `<${member.guild.id}>`);
  if (commander) {
    Logger.info("initiated by " + memberString(commander));
  }

  const guildId = member.guild.id;
  const takedownResource = createRandomAudioResource("takedowns");
  const failResource = createRandomAudioResource("fails");
  const wululuResource = createAudioResource(
    Readable.from(audioBuffers.WULULULULU as Buffer),
    {inputType: StreamType.OggOpus}
  );

  // update takedown lock if necessary and sync for every guild
  if (!takedownLocks[guildId]) takedownLocks[guildId] = new AwaitLock();
  const lock = takedownLocks[guildId];
  await lock.acquireAsync();
  Logger.debug("Acquired takedown lock for " + guildString(member.guild));

  if (!member.voice.channel) {
    lock.release();
    return {
      target: member,
      success: false,
      message: getRandom(replies.noChannel)
    };
  }

  if (!member.voice.channel.viewable) {
    Logger.warn(
      "cannot see channel " +
      member.voice.channel.name +
      " on " +
      guildString(member.guild)
    );
    lock.release();
    return {
      target: member,
      success: false,
      message: getRandom(replies.noChannel)
    };
  }

  if (!member.voice.channel.joinable) {
    Logger.warn(
      "cannot join channel " +
      member.voice.channel.name +
      " on " +
      guildString(member.guild)
    );
    lock.release();
    return {
      target: member,
      success: false,
      message: getRandom(replies.cantJoin)
    };
  }

  Logger.debug(`joining channel ${member.voice.channel.name}`);
  let connection: VoiceConnection;
  try {
    connection = joinVoiceChannel({
      channelId: member.voice.channel.id,
      guildId: guildId,
      // @ts-ignore I don't know what, seems turbo broken rn
      adapterCreator: member.guild.voiceAdapterCreator,
      selfDeaf: false,
      selfMute: false
    });
  }
  catch (e) {
    lock.release();
    return {
      target: member,
      success: false,
      message: getRandom(replies.cantJoin)
    };
  }


  let player: AudioPlayer
  try {
    player = createAudioPlayer();
    connection.subscribe(player);
    player.play(wululuResource);
  }
  catch (e) {
    lock.release();
    return {
      target: member,
      success: false,
      message: "can't talk here"
    };
  }

  // Calculate time of the wululu
  const wululuTime = 2000 + Math.floor(Math.random() * 3000);
  await sleep(wululuTime);

  // pause the player, kick the target and wait a little bit for the takedown
  // line
  player.pause();
  let takedownSuccess: boolean;
  try {
    Logger.debug("trying to kick member");
    await member.voice.disconnect();
    takedownSuccess = true;
    await sleep(500);
    player.play(takedownResource);
    let incStats = [Stats.TAKEDOWNS_SUCCESS, Stats.TIMES_TAKEN_DOWN];
    if (member.user.bot) incStats.push(Stats.BOTS_TAKEN_DOWN);
    if (commander) incStats.push(Stats.TIMES_COMMANDED_TAKEDOWN);
    incStat(incStats, member.guild, commander, member)
      .catch(console.error);
    Logger.info("takedown successful");
  }
  catch (error) {
    takedownSuccess = false;
    await sleep(800);
    player.play(failResource);
    incStat(Stats.TAKEDOWNS_FAILED, member.guild)
      .catch(console.error);
    Logger.warn("takedown failed");
  }

  // wait until the bot is done with playing the resource and destroy everything
  await Promise.any([
    pEvent(player, AudioPlayerStatus.Idle),
    sleep(20000)
  ]);
  player.stop();
  connection.destroy();
  wululuResource.playStream.destroy();
  takedownResource.playStream.destroy();
  failResource.playStream.destroy();
  lock.release();
  Logger.debug("released takedown lock for " + guildString(member.guild));
  return {target: member, success: takedownSuccess}
}

export default takedown;
