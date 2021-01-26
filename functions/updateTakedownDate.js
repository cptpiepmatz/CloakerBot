"use strict";

const dayjs = require("dayjs");

const readDatabase = require("./readDatabase");
const writeDatabase = require("./writeDatabase");

const collectionName = "random_takedowns";

/**
 * Updates the takedown date for a specific guild
 *
 * @param {Guild|Snowflake} guild
 * @param {Object} [guildDBData]
 * @returns {Promise<void>}
 */
async function updateTakedownDate(guild, guildDBData) {
  if (!guildDBData) {
    guildDBData = await readDatabase(collectionName, guild, {
      doRandomTakedowns: true
    });
  }

  // get min time for new takedown
  let times = [];
  times[0] = guildDBData.randomTakedownMin;
  if (typeof times[0] !== "number") {
    times[0] = 4;
  }

  // get max time for new takedown
  times[1] = guildDBData.randomTakedownMax;
  if (typeof times[1] !== "number") {
    times[1] = 24;
  }

  let minTime = Math.min(...times);
  let maxTime = Math.max(...times);

  // update the database with a new time
  let diffTime = minTime + (Math.random() * (maxTime - minTime));
  let newDate = dayjs().add(diffTime, "h").toDate();
  await writeDatabase(collectionName, guild, {takedownDate: newDate});
}

module.exports = updateTakedownDate;