/// <reference types="./types/global" />
import {Client, Intents, Interaction, Options} from "discord.js";

import token from "./provider/token";
import openDatabase from "./functions/database/openDatabase";
import GuildCommandInteraction from "./types/GuildCommandInteraction";
import RandomTakedownManager from "./structures/RandomTakedownManager";
import CommandManager from "./structures/CommandManager";

import takedownCommand from "./commands/takedown";
import randomCommand from "./commands/random";
import setRoleCommand from "./commands/takedownRole";
import ranksCommand from "./commands/ranks";
import helpCommand from "./commands/help";

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
    Intents.FLAGS.GUILD_INTEGRATIONS,
    Intents.FLAGS.DIRECT_MESSAGES
  ]
});

globalThis.takedownLocks = {};

globalThis.commandManager = new CommandManager();

client.once("ready", async client => {
  console.log("Started up the bot");
  await openDatabase();
  console.log("Opened the database");

  await commandManager.fetchCommands();
  for (let guild of client.guilds.cache.values()) {
    await commandManager.updatePermissions(guild);
  }

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

client.on("message", commandManager.initPermissions);

client.on("guildCreate", commandManager.updatePermissions);
client.on("guildIntegrationsUpdate", commandManager.updatePermissions);
client.on("guildUpdate", commandManager.updatePermissions);
client.on("roleCreate", commandManager.updatePermissions);
client.on("roleDelete", commandManager.updatePermissions);
client.on("roleUpdate", commandManager.updatePermissions);

client.login(token);
