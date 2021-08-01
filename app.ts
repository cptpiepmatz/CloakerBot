/// <reference types="./types/global" />
import {Client, Intents, Interaction} from "discord.js";

import token from "./provider/token";
import ResolvedNullError from "./error/ResolvedNullError";
import openDatabase from "./functions/database/openDatabase";
import GuildCommandInteraction from "./types/GuildCommandInteraction";

import takedownCommand from "./commands/takedown";
import randomCommand from "./commands/random";
import setRoleCommand from "./commands/takedownRole";
import ranksCommand from "./commands/ranks";
import helpCommand from "./commands/help";


globalThis.client = new Client({
  messageCacheLifetime: 3 * 60,
  messageSweepInterval: 30,
  intents: [
    Intents.FLAGS.GUILDS,
    Intents.FLAGS.GUILD_VOICE_STATES,
    Intents.FLAGS.GUILD_MESSAGES,
    Intents.FLAGS.GUILD_INTEGRATIONS
  ]
});

globalThis.takedownLocks = {};

client.once("ready", async () => {
  console.log("Started up the bot");
  await openDatabase();
  console.log("Opened the database");
  const testGuildId = "581884294495207444";
  let guild = await client.guilds.fetch(testGuildId);
  if (!guild) throw new ResolvedNullError(client.guilds, testGuildId);
  await guild.commands.set([
    takedownCommand.commandData,
    randomCommand.commandData,
    setRoleCommand.commandData,
    ranksCommand.commandData,
    helpCommand.commandData
  ]);
  console.log("Registered the commands");
  console.log("Everything done, let's go!");
});

client.on("interactionCreate", async (interaction: Interaction) => {
  if (!interaction.isCommand() || !interaction.inGuild()) return;
  const guildInteraction = interaction as GuildCommandInteraction;

  switch (interaction.commandName) {
    case "cloaker": return await takedownCommand(guildInteraction);
    case "random": return await randomCommand(guildInteraction);
    case "takedownrole": return await setRoleCommand(guildInteraction);
    case "ranks": return await ranksCommand(guildInteraction);
    case "help": return await helpCommand(guildInteraction);
  }
});

client.login(token);
