/// <reference types="./types/global" />
import {Client, Intents, Interaction} from "discord.js";

import {default as takedownCommand} from "./commands/takedown";
import random, {default as randomCommand} from "./commands/random";
import token from "./provider/token";
import ResolvedNullError from "./error/ResolvedNullError";
import openDatabase from "./functions/database/openDatabase";

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
    randomCommand.commandData
  ]);
  console.log("Registered the commands");
  console.log("Everything done, let's go!");
});

client.on("interactionCreate", async (interaction: Interaction) => {
  if (!interaction.isCommand() || !interaction.inGuild()) return;

  switch (interaction.commandName) {
    case "cloaker":
      await takedownCommand(interaction);
      break;
    case "random":
      await randomCommand(interaction);
      break;
  }
});

client.login(token);
