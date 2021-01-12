"use strict";

/**
 * Writes data to the database
 *
 * @param {string} collectionName
 * @param {Guild} guild
 * @param {object} data to insert into the database
 * @returns {Promise<void>}
 */
async function writeDatabase(collectionName, guild, data) {
  const collection = dbo.collection(collectionName);

  let insertData = {
    GuildID: guild.id,
    GuildName: guild.name
  };
  for (let dataKey in data) {
    if (data.hasOwnProperty(dataKey)) {
      insertData[dataKey] = data[dataKey];
    }
  }

  // uses the replaceOne to replace if the data is already in the db
  // adds the data if not existing
  await collection.replaceOne({GuildID: guild.id}, insertData, {upsert: true});
}

module.exports = writeDatabase;