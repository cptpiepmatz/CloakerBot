import {GuildMemberResolvable, GuildResolvable} from "discord.js";
import {join} from "path";
import {promises as fs} from "fs";
import {parse} from "json5";

import readDatabaseAggregation from "./database/readDatabaseAggregation";
import MemberRanks from "../types/MemberRanks";
import DatabaseTables from "../enums/DatabaseTables";

const pipelinePath = join(__dirname, "../aggregations/user-ranks.json5");

/**
 * Fetches the ranks for a guild member for a specific guild.
 * @param member Member to fetch the ranks for
 * @param guild Guild for the "here" ranks
 */
async function fetchMemberRanks(
    member: GuildMemberResolvable,
    guild: GuildResolvable
): Promise<null|MemberRanks> {
  let resolvedGuild = client.guilds.resolve(guild);
  if (!resolvedGuild) throw new Error("Couldn't fetch guild.");
  let resolvedMember = resolvedGuild.members.resolve(member);
  if (!resolvedMember) throw new Error("Couldn't fetch member.");

  let pipelineData = await fs.readFile(pipelinePath, {encoding: "utf8"});
  let pipeline = parse(
    pipelineData
      .replaceAll("%member%", resolvedMember.id)
      .replaceAll("%guild%", resolvedGuild.id)
  );
  let [rankedMember] = await readDatabaseAggregation(
    DatabaseTables.STATS_USERS, pipeline
  );
  return rankedMember as MemberRanks | null;
}

export default fetchMemberRanks;
