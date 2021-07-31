import RandomTakedownsDBData from "../../../types/database/RandomTakedownsDBData";
import getRandom from "../../../functions/util/getRandom";
import replies from "../../../provider/replies";
import GuildCommandInteraction from "../../../types/GuildCommandInteraction";

/**
 * Handler for the subcommand: "info".
 *
 * This will display the current state for random takedowns on this guild.
 * @param interaction Interaction of the command
 * @param dbData Old data from the database
 */
export default async function infoHandler(
  interaction: GuildCommandInteraction,
  dbData: RandomTakedownsDBData
): Promise<void> {
  if (dbData.doRandomTakedowns) {
    let minNumber = dbData.randomTakedownMin ?? "4 (default)";
    let maxNumber = dbData.randomTakedownMax ?? "24 (default)";
    await interaction.editReply(
      "*Currently the random takedown checks take between "
      + `${minNumber} and ${maxNumber} hours.* \n`
      + getRandom(replies.doRandom)
    );
  }
  else {
    await interaction.editReply(getRandom(replies.doNotRandom));
  }
}
