import {ApplicationCommandData, MessageEmbed} from "discord.js";

import colors from "../provider/colors";
import helpText from "../provider/helpText";
import GuildCommandInteraction from "../types/GuildCommandInteraction";
import constructCommand from "../functions/util/constructCommand";

const embed = new MessageEmbed()
  .setColor(colors.primaryColor)
  .setTitle(helpText.title)
  .setDescription(helpText.description)
  .addFields(helpText.fields);

async function helpCommand(interaction: GuildCommandInteraction) {
  await interaction.reply({embeds: [embed], ephemeral: true});
}

const commandData: ApplicationCommandData = {
  name: "help",
  description: "Shows a help page."
}

export default constructCommand(helpCommand, commandData);
