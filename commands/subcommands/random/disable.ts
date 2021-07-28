import GuildCommandInteraction from "../../../types/GuildCommandInteraction";
import RandomTakedownsDBData from "../../../types/database/RandomTakedownsDBData";
import writeDatabase from "../../../functions/database/writeDatabase";
import DatabaseTables from "../../../enums/DatabaseTables";
import getRandom from "../../../functions/util/getRandom";
import replies from "../../../provider/replies";
import createUndoButton from "../../util/random/createUndoButton";

/**
 * Handler for the subcommand: "disable".
 * This will disable the random takedowns on this guild.
 * @param interaction Interaction of the command
 * @param dbData Old data from the database
 */
export default async function disableHandler(
  interaction: GuildCommandInteraction,
  dbData: RandomTakedownsDBData
): Promise<void> {
  await writeDatabase(
    DatabaseTables.RANDOM_TAKEDOWNS,
    interaction.guild.id,
    {doRandomTakedowns: true}
  );
  await interaction.editReply(getRandom(replies.doNotRandom));
  await createUndoButton(interaction, dbData);
}
