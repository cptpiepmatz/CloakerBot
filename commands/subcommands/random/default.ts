import GuildCommandInteraction from "../../../types/GuildCommandInteraction";
import RandomTakedownsDBData from "../../../types/database/RandomTakedownsDBData";
import clearDatabase from "../../../functions/database/clearDatabase";
import DatabaseTables from "../../../enums/DatabaseTables";
import createUndoButton from "../../util/random/createUndoButton";

/**
 * Handler function for the subcommand: "default".
 * This resets the database entry for this guild and therefore effectively sets
 * the default value.
 * @param interaction Interaction of the command
 * @param dbData Old data from the database
 */
export default async function defaultHandler(
  interaction: GuildCommandInteraction,
  dbData: RandomTakedownsDBData
): Promise<void> {
  await clearDatabase(
    DatabaseTables.RANDOM_TAKEDOWNS,
    interaction.guild
  );
  await interaction.editReply("Everything back to normal, I guess.");
  await createUndoButton(interaction, dbData);
}
