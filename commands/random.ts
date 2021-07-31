import {ApplicationCommandData, CommandInteraction, Guild} from "discord.js";

import constructCommand from "../functions/util/constructCommand";
import DatabaseTables from "../enums/DatabaseTables";
import readDatabase from "../functions/database/readDatabase";
import RandomTakedownsDBData from "../types/database/RandomTakedownsDBData";
import GuildCommandInteraction from "../types/GuildCommandInteraction";
import infoHandler from "./subcommands/random/info";
import defaultHandler from "./subcommands/random/default";
import setHandler from "./subcommands/random/set";

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
    case "default": return await defaultHandler(guildInteraction, dbData);
    case "set": return await setHandler(guildInteraction, dbData);
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
      type: "SUB_COMMAND",
      description: "Set values for the random takedowns",
      options: [
        {
          name: "enable",
          type: "STRING",
          choices: [
            {
              name: "enable",
              value: "enable"
            },
            {
              name: "disable",
              value: "disable"
            }
          ],
          description:
            "Set if you want to enable or disable the random takedowns",
        },
        {
          name: "max",
          type: "STRING",
          description:
            "Set the UPPER end of the frequency in hours (Example: 0.25)",
        },
        {
          name: "min",
          type: "STRING",
          description:
            "Set the LOWER end of the frequency in hours (Example: 0.25)",
        }
      ]
    },
    {
      name: "default",
      type: "SUB_COMMAND",
      description: "Reset the random settings for this server"
    }
  ]
}

export default constructCommand(randomCommand, commandData);
