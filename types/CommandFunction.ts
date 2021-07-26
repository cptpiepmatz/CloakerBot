import {ApplicationCommandData, CommandInteraction} from "discord.js";

/** Type for functions to execute commands. */
type CommandFunction = {
  commandData: ApplicationCommandData;
  (interaction: CommandInteraction): Promise<void>
}

export default CommandFunction;
