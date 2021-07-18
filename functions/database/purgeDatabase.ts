import {GuildResolvable} from "discord.js";

/**
 * Removes all database entries for a guild.
 * @param guild Guild to purge from the database
 */
async function purgeDatabase(guild: GuildResolvable): Promise<void> {
  let collections = await globalThis.dbo.collections();
  let resolvedGuild = globalThis.client.guilds.resolve(guild);
  for (let collection of collections) {
    await collection.deleteOne({GuildID: resolvedGuild.id});
  }
  console.log("Wiped database entries for " + resolvedGuild.name);
}

export default purgeDatabase;
