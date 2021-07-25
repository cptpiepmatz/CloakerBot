import CommandFunction from "../types/CommandFunction";
import {ApplicationCommandData, Interaction} from "discord.js";

import constructCommand from "../functions/util/constructCommand";

async function takedownCommand(interaction: Interaction) {
  
}

let commandData = {
  name: "cloaker",
  description: "Takes down your friends"
}

export default constructCommand(takedownCommand, commandData);
