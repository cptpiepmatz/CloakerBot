import GuildCommandInteraction from "../../../types/GuildCommandInteraction";
import RandomTakedownsDBData from "../../../types/database/RandomTakedownsDBData";
import getRandom from "../../../functions/util/getRandom";
import replies from "../../../provider/replies";
import writeDatabase from "../../../functions/database/writeDatabase";
import DatabaseTables from "../../../enums/DatabaseTables";
import createUndoButton from "../../util/random/createUndoButton";

/**
 * Handler for the subcommand: "min".
 * This will set the min value for the frequency of random takedowns.
 * @param interaction Interaction of the command
 * @param dbData Old data from the database
 */
export default async function minHandler(
  interaction: GuildCommandInteraction,
  dbData: RandomTakedownsDBData
): Promise<void> {
  let minNumber = Number.parseFloat(interaction.options.getString("hours", true));
  if (Number.isNaN(minNumber)) {
    await interaction.editReply(getRandom(replies.noNumber));
    return;
  }

  await writeDatabase(DatabaseTables.RANDOM_TAKEDOWNS, interaction.guild, {
    doRandomTakedowns: true,
    randomTakedownMin: minNumber
  });
  await interaction.editReply(
    "*The minimum time between random takedown checks was set to" +
    ` ${minNumber} hours.*`
  );
  await createUndoButton(interaction, dbData);
}
