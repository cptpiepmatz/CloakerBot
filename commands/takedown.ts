import CommandFunction from "../types/CommandFunction";
import {ApplicationCommandData, CommandInteraction, GuildMember, Interaction} from "discord.js";

import constructCommand from "../functions/util/constructCommand";
import takedown from "../functions/takedown";

async function takedownCommand(interaction: CommandInteraction) {
  let member = interaction.options.get("user")?.member;
  if (member instanceof GuildMember) {
    await takedown(member);
  }
}

const commandData: ApplicationCommandData = {
  name: "cloaker",
  description: "Take down your friends",
  options: [{
    name: "user",
    type: "USER",
    description: "User to take down",
    required: true
  }]
};

export default constructCommand(takedownCommand, commandData);
