import {
 ApplicationCommandData,
 ApplicationCommandPermissionData,
 ChatInputApplicationCommandData,
 Guild
} from "discord.js";
import CommandFunction from "../../types/CommandFunction";
import GuildCommandInteraction from "../../types/GuildCommandInteraction";

/**
 * Utility function to construct the command functions for export.
 *
 * @param rawCommandFunction The raw command to execute while running the command
 * @param commandData The command data being passed to the application manager
 * @param getPermissions Permission getter if default permissions are set to false
 */
export default function constructCommand(
  rawCommandFunction: (interaction: GuildCommandInteraction) => Promise<void>,
  commandData: ChatInputApplicationCommandData,
  getPermissions?: (guild: Guild) => Promise<ApplicationCommandPermissionData[]>
): CommandFunction {
 const commandFunction = Object.assign(
   rawCommandFunction,
   {commandData: commandData}
 );
 if (getPermissions) {
  return Object.assign(commandFunction, {getPermissions: getPermissions});
 }
 return commandFunction;
}
