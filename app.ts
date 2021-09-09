/// <reference types="./types/global" />
import {Client, Intents, Interaction, Options} from "discord.js";

import token from "./provider/token";
import openDatabase from "./functions/database/openDatabase";
import GuildCommandInteraction from "./types/GuildCommandInteraction";
import RandomTakedownManager from "./structures/RandomTakedownManager";
import CommandManager from "./structures/CommandManager";
import OwnerNotifier from "./structures/OwnerNotifier";
import DangerZoneManager from "./structures/DangerZoneManager";

import takedownCommand from "./commands/takedown";
import randomCommand from "./commands/random";
import setRoleCommand from "./commands/takedownRole";
import ranksCommand from "./commands/ranks";
import helpCommand from "./commands/help";
import dangerZoneCommand from "./commands/dangerZone";
import Logger from "js-logger";
import createLogHandler from "./functions/createLogHandler";
import sleep from "sleep-promise";
import {inspect} from "util";

Logger.useDefaults();
Logger.setHandler(createLogHandler);
Logger.setLevel(Logger.INFO);

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
Logger.trace("created client");

globalThis.takedownLocks = {};
Logger.trace("created takedown locks");

globalThis.commandManager = new CommandManager();
Logger.trace("created command manager");

client.once("ready", async client => {
  Logger.debug("client is ready");

  await openDatabase();
  Logger.debug("opened the database");

  await commandManager.fetchCommands();
  Logger.debug("fetched commands");

  const ownerNotifier = await OwnerNotifier.create();
  client.on("messageCreate", ownerNotifier.messageOwner.bind(ownerNotifier));
  client.on("interactionCreate", ownerNotifier.unsubscribe.bind(ownerNotifier));
  client.on("interactionCreate", ownerNotifier.subscribe.bind(ownerNotifier));
  Logger.debug("created owner notifier and bound events");

  globalThis.dangerZoneManager = await DangerZoneManager.create();
  client.on("voiceStateUpdate", dangerZoneManager.controlZone.bind(dangerZoneManager));
  Logger.debug("created danger zone manager and bound events");

  new RandomTakedownManager(30000).startInterval();
  Logger.debug("started random takedown manager");

  Logger.info("booted up!");
});

client.on("interactionCreate", async (interaction: Interaction) => {
  Logger.debug("received interaction");

  if (!interaction.isCommand() || !interaction.inGuild()) return;
  const guildInteraction = interaction as GuildCommandInteraction;

  Logger.debug("interaction is command and in guild");
  Logger.debug(`guild: ${guildInteraction.guild.name}`);
  Logger.debug(`member: ${guildInteraction.member?.user.username}#${guildInteraction.member?.user.discriminator}`);
  Logger.debug(`command: ${interaction.commandName}`);
  Logger.debug("options:");
  for (let option of interaction.options["_hoistedOptions"]) {
    Logger.debug(`[${option.type}] ${option.name}: ${option.value}`);
  }

  Logger.info(
    "executing command '" +
    interaction.commandName +
    "' on '" +
    guildInteraction.guild.name +
    "' by '" +
    guildInteraction.member?.user.username +
    "#" +
    guildInteraction.member?.user.discriminator +
    "'"
  );
  try {
    switch (interaction.commandName) {
      case takedownCommand.commandData.name: return await takedownCommand(guildInteraction);
      case randomCommand.commandData.name: return await randomCommand(guildInteraction);
      case setRoleCommand.commandData.name: return await setRoleCommand(guildInteraction);
      case ranksCommand.commandData.name: return await ranksCommand(guildInteraction);
      case helpCommand.commandData.name: return await helpCommand(guildInteraction);
      case dangerZoneCommand.commandData.name: return await dangerZoneCommand(guildInteraction);
    }
  }
  catch (e) {
    Logger.error(e);
  }
});

client.on("messageCreate", commandManager.initPermissions.bind(commandManager));

client.on("guildCreate", commandManager.updatePermissions.bind(commandManager));
client.on("guildIntegrationsUpdate", commandManager.updatePermissions.bind(commandManager));
client.on("guildUpdate", commandManager.updatePermissions.bind(commandManager));
client.on("roleCreate", commandManager.updatePermissions.bind(commandManager));
client.on("roleDelete", commandManager.updatePermissions.bind(commandManager));
client.on("roleUpdate", commandManager.updatePermissions.bind(commandManager));

Logger.debug("logging in...");
client.login(token);
