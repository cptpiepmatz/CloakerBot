import {ApplicationCommandData, CommandInteraction, Guild} from "discord.js";

import constructCommand from "../functions/util/constructCommand";
import DatabaseTables from "../enums/DatabaseTables";
import readDatabase from "../functions/database/readDatabase";
import RandomTakedownsDBData from "../types/database/RandomTakedownsDBData";
import GuildCommandInteraction from "../types/GuildCommandInteraction";
import infoHandler from "./subcommands/random/info";
import enableHandler from "./subcommands/random/enable";
import disableHandler from "./subcommands/random/disable";
import maxHandler from "./subcommands/random/max";
import minHandler from "./subcommands/random/min";
import defaultHandler from "./subcommands/random/default";

/**
 * Command handling everything about random takedowns.
 * @param interaction
 */
async function randomCommand(interaction: CommandInteraction): Promise<void> {
  if (!(interaction.guild instanceof Guild)) return;
  let guildInteraction = interaction as GuildCommandInteraction;

  let subCommand = interaction.options.getSubCommand(true);
  let dbData = (await readDatabase(
    DatabaseTables.RANDOM_TAKEDOWNS,
    interaction.guild
  ) ?? {doRandomTakedowns: false}) as RandomTakedownsDBData;

  await interaction.defer();
  switch (subCommand) {
    case "info": return await infoHandler(guildInteraction, dbData);
    case "enable": return await enableHandler(guildInteraction, dbData);
    case "disable": return await disableHandler(guildInteraction, dbData);
    case "max": return await maxHandler(guildInteraction, dbData);
    case "min": return await minHandler(guildInteraction, dbData);
    case "default": return await defaultHandler(guildInteraction, dbData);
  }
}

const commandData: ApplicationCommandData = {
  name: "random",
  description: "Check or set the random takedowns for this guild",
  options: [
    {
      name: "info",
      type: "SUB_COMMAND",
      description: "Shows the current state for random takedowns",
    },
    {
      name: "set",
      type: "SUB_COMMAND_GROUP",
      description: "Set values for the random takedowns",
      options: [
        {
          name: "enable",
          type: "SUB_COMMAND",
          description: "Enables random takedowns on this server"
        },
        {
          name: "disable",
          type: "SUB_COMMAND",
          description: "Disables random takedowns on this server"
        },
        {
          name: "max",
          type: "SUB_COMMAND",
          description: "Set the UPPER end of the frequency for random takedowns",
          options: [{
            name: "hours",
            type: "STRING",
            required: true,
            description: "Set the time in hours. (Example: *0.25*)"
          }]
        },
        {
          name: "min",
          type: "SUB_COMMAND",
          description: "Set the LOWER end of the frequency for random takedowns",
          options: [{
            name: "hours",
            type: "STRING",
            required: true,
            description: "Set the time in hours. (Example: *0.25*)"
          }]
        },
        {
          name: "default",
          type: "SUB_COMMAND",
          description: "Reset the random settings for this server"
        }
      ]
    }
  ]
}

export default constructCommand(randomCommand, commandData);
