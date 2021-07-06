"use strict";

const Discord = require("discord.js");
const json5 = require("json5");
const fs = require("fs/promises");
const path = require("path");

const readDatabaseAggregation = require("./database/readDatabaseAggregation");

const pipelinePath = path.join(__dirname, "../aggregations/user-ranks.json5");

/**
 * Fetches the ranks for a guild member for a specific guild.
 *
 * @param {GuildMember|User|Snowflake} member
 * @param {Guild|Snowflake} guild
 * @return {Promise<null|{
 *                         _id: Snowflake,
 *                         UserID: Snowflake,
 *                         UserName: string,
 *                         victimRankHere: number,
 *                         victimRankTotal: number,
 *                         punisherRankHere: number,
 *                         punisherRankTotal: number,
 *                         timesSmackedHere: number,
 *                         timesSmackedTotal: number,
 *                         timesCalledHere: number,
 *                         timesCalledTotal: number
 *                       }>}
 */
async function fetchMemberRanks(member, guild) {
  let memberId = member.id || member;
  let guildId = guild.id || guild;

  let pipelineData = await fs.readFile(pipelinePath, {encoding: "utf8"});
  let pipeline = json5.parse(
    pipelineData
      .replaceAll("%member%", memberId)
      .replaceAll("%guild%", guildId)
  );
  let [rankedMember] = await readDatabaseAggregation("stats_users", pipeline);
  return rankedMember;
}

module.exports = fetchMemberRanks;
