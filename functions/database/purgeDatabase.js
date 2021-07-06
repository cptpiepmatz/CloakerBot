"use strict";

/**
 * Removes all database entries for a guild
 *
 * @param {Guild} guild
 * @returns {Promise<object>}
 */
async function purgeDatabase(guild) {
  let collections = await dbo.collections();
  for (let collection of collections) {
    await collection.deleteOne({GuildID: guild.id});
  }
  console.log("Wiped database entries for " + guild.name);
}

module.exports = purgeDatabase;