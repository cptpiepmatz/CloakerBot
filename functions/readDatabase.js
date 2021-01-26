"use strict";

/**
 * Reads data from the db for a specific guild
 *
 * @param {string} collectionName
 * @param {Guild|"all"} guild
 * @param {object} [query]
 * @returns {Promise<object>|Promise<object[]>}
 */
async function readDatabase(collectionName, guild, query) {
  const collection = dbo.collection(collectionName);

  if (guild !== "all") {
    return await collection.findOne({GuildID: guild.id});
  }

  let cursor = collection.find(query);
  let returnData = await cursor.toArray();
  cursor.close();
  return returnData;
}

module.exports = readDatabase;