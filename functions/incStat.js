"use strict";

const guildCollection = "stats_guilds";
const userCollection = "stats_users";

/**
 * Function to increase stats
 *
 * @param {String|String[]|incStat.STATS|incStat.STATS[]} statsToInc - stats to increment in the database
 * @param {Guild} guild where the stat increase happened
 * @param {?GuildMember} [commander=null] who commanded the takedown
 * @param {?GuildMember} [target=null] who was taken down down
 * @returns {Promise<void>}
 */
async function incStat(statsToInc, guild, commander = null, target = null) {

}

module.exports = incStat;