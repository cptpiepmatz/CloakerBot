import {
  ChatInputApplicationCommandData,
  Guild,
  ApplicationCommandPermissionData
} from "discord.js";

import GuildCommandInteraction from "./GuildCommandInteraction";

/** Type for functions to execute commands. */
type CommandFunction = {
  commandData: ChatInputApplicationCommandData;
  (interaction: GuildCommandInteraction): Promise<void>;
  getPermissions?: (guild: Guild) => Promise<ApplicationCommandPermissionData[]>;
}

export default CommandFunction;
