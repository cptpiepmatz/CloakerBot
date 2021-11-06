import {
  Message,
  MessageActionRow,
  MessageButton,
  MessageComponentInteraction,
  MessageEmbed,
  SnowflakeUtil
} from "discord.js";

import GuildCommandInteraction from "../../../types/GuildCommandInteraction";
import RandomTakedownsDBData from "../../../types/database/RandomTakedownsDBData";
import writeDatabase from "../../../functions/database/writeDatabase";
import DatabaseTables from "../../../enums/DatabaseTables";
import colors from "../../../provider/colors";
import clearDatabase from "../../../functions/database/clearDatabase";
import Logger from "js-logger";

/**
 * A util function for the random command to create an "undo" button.
 * @param interaction Interaction where the button should appear
 * @param dbData The old data from the database
 */
export default async function createUndoButton(
  interaction: GuildCommandInteraction,
  dbData: RandomTakedownsDBData
): Promise<void> {

  const customId = SnowflakeUtil.generate();
  const row = new MessageActionRow().addComponents(
    new MessageButton()
      .setCustomId(customId)
      .setLabel("Undo")
      .setStyle("DANGER")
  );
  await interaction.editReply({components: [row]});

  /**
   * Filter method. Filters that only the first commander is allowed.
   * Also it compares for the correct custom id.
   * @param interactionToFilter The interaction the filter runs against
   */
  async function filter(interactionToFilter: MessageComponentInteraction) {
    if (interactionToFilter.user.id !== interaction.user.id) {
      await interactionToFilter.reply({
        ephemeral: true,
        content: "You're not allowed to press this button."
      });
      return false;
    }
    await interactionToFilter.deferReply();
    return interactionToFilter.customId === customId;
  }

  try {
    let message = await interaction.fetchReply() as Message;
    let buttonInteraction = await message.awaitMessageComponent({
      filter,
      componentType: "BUTTON",
      time: 60 * 3 * 1000
    });

    await clearDatabase(DatabaseTables.RANDOM_TAKEDOWNS, interaction.guild);
    await writeDatabase(
      DatabaseTables.RANDOM_TAKEDOWNS, interaction.guild, dbData);

    let updateValues = "";
    for (let [key, value] of Object.entries(dbData)) {
      if (!["_id", "GuildID", "GuildName", "takedownDate"].includes(key)) {
        updateValues += `${key}: ${value}\n`
      }
    }

    const undoEmbed = new MessageEmbed()
      .setTitle("Undo successful!")
      .addField("Values in the DB now:", updateValues)
      .setColor(colors.primaryColor);
    await buttonInteraction?.editReply({embeds: [undoEmbed]});
  }
  catch (e) {
    Logger.error(e);
    process.exit(1);
  }
  try {
    row.components[0].setDisabled(true);
    await interaction.editReply({components: [row]});
  }
  catch (e) {
    Logger.error(e);
    process.exit(1);
  }
}
