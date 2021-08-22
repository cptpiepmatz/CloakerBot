/// <reference types="./types/global" />
import {ApplicationCommandData, Client, Intents, Interaction, Options} from "discord.js";

import token from "./provider/token";
import ResolvedNullError from "./error/ResolvedNullError";
import openDatabase from "./functions/database/openDatabase";
import GuildCommandInteraction from "./types/GuildCommandInteraction";

import takedownCommand from "./commands/takedown";
import randomCommand from "./commands/random";
import setRoleCommand from "./commands/takedownRole";
import ranksCommand from "./commands/ranks";
import helpCommand from "./commands/help";
import RandomTakedownManager from "./structures/RandomTakedownManager";
import CommandManager from "./structures/CommandManager";

globalThis.client = new Client({
  makeCache: Options.cacheWithLimits({
    MessageManager: {
      sweepInterval: 60 * 3,
      maxSize: 50
    },
    ThreadManager: 50
  }),
  intents: [
    Intents.FLAGS.GUILDS,
    Intents.FLAGS.GUILD_VOICE_STATES,
    Intents.FLAGS.GUILD_MESSAGES,
    Intents.FLAGS.GUILD_INTEGRATIONS
  ]
});

globalThis.takedownLocks = {};

client.once("ready", async client => {
  console.log("Started up the bot");
  await openDatabase();
  console.log("Opened the database");

  const commandManager = new CommandManager();
  await commandManager.fetchCommands();

  process.exit(0);

  new RandomTakedownManager(120000).startInterval();
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
