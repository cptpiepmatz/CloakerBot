import {ApplicationCommandData, Interaction} from "discord.js";

/** Type for functions to execute commands. */
type CommandFunction = {
  commandData: ApplicationCommandData;
  (interaction: Interaction): Promise<void>
}

export default CommandFunction;
