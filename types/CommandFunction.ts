import {ApplicationCommandData, ChatInputApplicationCommandData} from "discord.js";

import GuildCommandInteraction from "./GuildCommandInteraction";

/** Type for functions to execute commands. */
type CommandFunction = {
  commandData: ChatInputApplicationCommandData;
  (interaction: GuildCommandInteraction): Promise<void>
}

export default CommandFunction;
