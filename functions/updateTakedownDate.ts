import {GuildResolvable} from "discord.js";
import dayjs = require("dayjs");

import DatabaseTables from "../enums/DatabaseTables";
import RandomTakedownsDBData from "../types/database/RandomTakedownsDBData";
import readDatabase from "./database/readDatabase";
import writeDatabase from "./database/writeDatabase";

const collectionName = DatabaseTables.RANDOM_TAKEDOWNS;

/**
 * Updates the takedown date for a specific guild.
 * @param guild
 * @param guildDBData
 */
async function updateTakedownDate(
    guild: GuildResolvable,
    guildDBData?: RandomTakedownsDBData
): Promise<void> {
  if (!guildDBData) {
    guildDBData = await readDatabase(collectionName, guild, {
      doRandomTakedowns: true
    }) as RandomTakedownsDBData;
  }

  // get min time for new takedown
  let times = [NaN, NaN];
  times[0] = guildDBData.randomTakedownMin ?? 24;

  // get max time for new takedown
  times[1] = guildDBData.randomTakedownMax ?? 24;

  let minTime = Math.min(...times);
  let maxTime = Math.max(...times);

  // update the database with a new time
  let diffTime = minTime + (Math.random() * (maxTime - minTime));
  let newDate = dayjs().add(diffTime, "h").toDate();
  await writeDatabase(collectionName, guild, {takedownDate: newDate});
}

export default updateTakedownDate;
