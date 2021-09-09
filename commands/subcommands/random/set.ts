import GuildCommandInteraction from "../../../types/GuildCommandInteraction";
import RandomTakedownsDBData from "../../../types/database/RandomTakedownsDBData";
import writeDatabase from "../../../functions/database/writeDatabase";
import DatabaseTables from "../../../enums/DatabaseTables";
import createUndoButton from "../../util/random/createUndoButton";

/**
 * Handler for the subcommand: "set".
 *
 * This will handle change in settings for the random takedown.
 * @param interaction Interaction of the command
 * @param dbData Old data from the database
 */
export default async function setHandler(
  interaction: GuildCommandInteraction,
  dbData: RandomTakedownsDBData
): Promise<void> {
  const enable = interaction.options.get("enable")?.value;
  const max = parseFloat(interaction.options.getString("max") ?? "") || null;
  const min = parseFloat(interaction.options.getString("min") ?? "") || null;

  if (!(enable || max || min)) {
    await interaction.editReply("*Enter at least one parameter.*");
    return;
  }

  let writeObject: Partial<RandomTakedownsDBData> = {};
  if (enable) writeObject.doRandomTakedowns = enable === "enable";
  if (max) writeObject.randomTakedownMax = max;
  if (min) writeObject.randomTakedownMin = min;
  writeObject.takedownDate = undefined;

  await writeDatabase(
    DatabaseTables.RANDOM_TAKEDOWNS,
    interaction.guild,
    writeObject
  );

  let replyMessage = "***You set the following:**";
  if (enable) replyMessage += "\nDo random takedowns: " + enable;
  if (max) replyMessage += `\nMax time until takedown: ${max} hours`;
  if (min) replyMessage += `\nMin time until takedown: ${min} hours`;
  replyMessage += "*";
  await interaction.editReply(replyMessage);

  await createUndoButton(interaction, dbData);
}
