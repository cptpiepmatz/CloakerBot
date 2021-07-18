import {GuildResolvable} from "discord.js";
import DatabaseTables from "../../enums/DatabaseTables";

/**
 * Clears data from the database for a specific guild.
 * Only the specified collection will be cleared.
 * @param collectionName Collection to clear of
 * @param guild Guild to clear for
 */
async function clearDatabase(collectionName: DatabaseTables, guild: GuildResolvable) {
  const collection = globalThis.dbo.collection(collectionName);
  let resolvedGuild = globalThis.client.guilds.resolve(guild);
  if (!resolvedGuild) throw new Error("Couldn't fetch guild.");
  let guildId = resolvedGuild.id;
  await collection.deleteOne({GuildID: guildId});
}

export default clearDatabase;
