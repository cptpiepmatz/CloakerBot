import {ApplicationCommandData, Interaction} from "discord.js";
import CommandFunction from "../../types/CommandFunction";

/**
 * Utility function to construct the command functions for export.
 *
 * @param rawCommandFunction The raw command to execute while running the command
 * @param commandData The command data being passed to the application manager
 */
export default function constructCommand(
  rawCommandFunction: (interaction: Interaction) => Promise<void>,
  commandData: ApplicationCommandData
): CommandFunction {
 return Object.assign(rawCommandFunction, {commandData: commandData});
}
