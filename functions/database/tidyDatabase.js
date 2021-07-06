"use strict";

const readDatabase = require("./readDatabase");
const updateTakedownDate = require("../updateTakedownDate");

/**
 * Tidies the database, good if old database needs some updates
 *
 * @returns {Promise<void>}
 */
async function tidyDatabase() {
  await tidyRandomTakedowns();
}

/**
 * Subroutine for the tidying up the database.
 * Checks the table random_takedowns
 *
 * @returns {Promise<void>}
 */
async function tidyRandomTakedowns() {
  let guildData = await readDatabase(
    "random_takedowns",
    "all",
    {
      doRandomTakedowns: true,
      takedownDate: null
    });

  for (let guildDate of guildData) {
    await updateTakedownDate(guildDate.GuildID, guildDate);
  }
}

module.exports = tidyDatabase;
