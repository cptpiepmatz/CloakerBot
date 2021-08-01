import {ApplicationCommandData} from "discord.js";

import GuildCommandInteraction from "./GuildCommandInteraction";

/** Type for functions to execute commands. */
type CommandFunction = {
  commandData: ApplicationCommandData;
  (interaction: GuildCommandInteraction): Promise<void>
}

export default CommandFunction;
