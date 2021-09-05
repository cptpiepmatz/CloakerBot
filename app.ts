/// <reference types="./types/global" />
import {Client, Intents, Interaction, Options} from "discord.js";

import token from "./provider/token";
import openDatabase from "./functions/database/openDatabase";
import GuildCommandInteraction from "./types/GuildCommandInteraction";
import RandomTakedownManager from "./structures/RandomTakedownManager";
import CommandManager from "./structures/CommandManager";
import OwnerNotifier from "./structures/OwnerNotifier";
import update200 from "./owner_messages/Update 2.0.0";
import DangerZoneManager from "./structures/DangerZoneManager";

import takedownCommand from "./commands/takedown";
import randomCommand from "./commands/random";
import setRoleCommand from "./commands/takedownRole";
import ranksCommand from "./commands/ranks";
import helpCommand from "./commands/help";
import dangerZoneCommand from "./commands/dangerZone";

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

  const ownerNotifier = await OwnerNotifier.create();
  client.on("messageCreate", ownerNotifier.messageOwner.bind(ownerNotifier));
  client.on("interactionCreate", ownerNotifier.unsubscribe.bind(ownerNotifier));
  client.on("interactionCreate", ownerNotifier.subscribe.bind(ownerNotifier));

  globalThis.dangerZoneManager = await DangerZoneManager.create();
  client.on("voiceStateUpdate", dangerZoneManager.controlZone.bind(dangerZoneManager));

  new RandomTakedownManager(120000).startInterval();
  console.log("Everything done, let's go!");
});

client.on("interactionCreate", async (interaction: Interaction) => {
  if (!interaction.isCommand() || !interaction.inGuild()) return;
  const guildInteraction = interaction as GuildCommandInteraction;

  switch (interaction.commandName) {
    case takedownCommand.commandData.name: return await takedownCommand(guildInteraction);
    case randomCommand.commandData.name: return await randomCommand(guildInteraction);
    case setRoleCommand.commandData.name: return await setRoleCommand(guildInteraction);
    case ranksCommand.commandData.name: return await ranksCommand(guildInteraction);
    case helpCommand.commandData.name: return await helpCommand(guildInteraction);
    case dangerZoneCommand.commandData.name: return await dangerZoneCommand(guildInteraction);
  }
});

client.on("messageCreate", commandManager.initPermissions.bind(commandManager));

client.on("guildCreate", commandManager.updatePermissions.bind(commandManager));
client.on("guildIntegrationsUpdate", commandManager.updatePermissions.bind(commandManager));
client.on("guildUpdate", commandManager.updatePermissions.bind(commandManager));
client.on("roleCreate", commandManager.updatePermissions.bind(commandManager));
client.on("roleDelete", commandManager.updatePermissions.bind(commandManager));
client.on("roleUpdate", commandManager.updatePermissions.bind(commandManager));

client.login(token);
