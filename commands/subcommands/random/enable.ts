import GuildCommandInteraction from "../../../types/GuildCommandInteraction";
import RandomTakedownsDBData from "../../../types/database/RandomTakedownsDBData";
import writeDatabase from "../../../functions/database/writeDatabase";
import DatabaseTables from "../../../enums/DatabaseTables";
import updateTakedownDate from "../../../functions/updateTakedownDate";
import getRandom from "../../../functions/util/getRandom";
import replies from "../../../provider/replies";
import createUndoButton from "../../util/random/createUndoButton";

/**
 * Handler for the subcommand: "enable".
 * This will enable the random takedowns on this guild.
 * @param interaction Interaction of the command
 * @param dbData Old data from the database
 */
export default async function enableHandler(
  interaction: GuildCommandInteraction,
  dbData: RandomTakedownsDBData
): Promise<void> {
  await writeDatabase(
    DatabaseTables.RANDOM_TAKEDOWNS,
    interaction.guild.id,
    {doRandomTakedowns: true}
  );
  await updateTakedownDate(interaction.guild);
  await interaction.editReply(getRandom(replies.doRandom));
  await createUndoButton(interaction, dbData);
}
