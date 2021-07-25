import readDatabase from "./readDatabase";
import DatabaseTables from "../../enums/DatabaseTables";
import RandomTakedownsDBData from "../../types/database/RandomTakedownsDBData";
import updateTakedownDate from "../updateTakedownDate";

/**
 * Tidies the database, good if old database needs some updates
 */
async function tidyDatabase(): Promise<void> {
  await tidyRandomTakedowns();
}

/**
 * Subroutine for the tidying up the database.
 * Checks the table random_takedowns
 */
async function tidyRandomTakedowns(): Promise<void> {
  let guildData = await readDatabase(
      DatabaseTables.RANDOM_TAKEDOWNS,
      "all",
      {
        doRandomTakedowns: true,
        takedownDate: null
      }
  ) as RandomTakedownsDBData[];

  for (let guildDate of guildData) {
    await updateTakedownDate(guildDate.GuildID, guildDate);
  }
}

export default tidyRandomTakedowns;
