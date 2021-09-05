import {Guild, GuildResolvable} from "discord.js";
import {Document} from "mongodb";
import DatabaseTables from "../../enums/DatabaseTables";
import ResolvedNullError from "../../error/ResolvedNullError";

async function readDatabase(
  collectionName: DatabaseTables,
  guild: GuildResolvable,
  query?: Document
): Promise<object | undefined>;
async function readDatabase(
  collectionName: DatabaseTables,
  guild: "all",
  query?: Document
): Promise<object[]>
/**
 * Reads data from the database for a specific or all guilds.
 * @param collectionName The collection to read from
 * @param guild The guild to read for or "all"
 * @param query The query to with "all"
 */
async function readDatabase(
    collectionName: DatabaseTables,
    guild: GuildResolvable | "all",
    query?: Document
): Promise<object | object[] | undefined> {
  const collection = dbo.collection(collectionName);

  if (guild !== "all") {
    let resolvedGuild = client.guilds.resolve(guild);
    if (!resolvedGuild) throw new ResolvedNullError(guild, client.guilds);
    return await collection.findOne({GuildID: resolvedGuild.id});
  }

  let cursor = query ? collection.find(query) : collection.find();
  let returnData = await cursor.toArray();
  cursor.close();
  return returnData;
}

export default readDatabase;
