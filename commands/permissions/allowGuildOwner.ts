import {ApplicationCommandPermissionData, Guild} from "discord.js";

export default function allowGuildOwner(
  guild: Guild
): ApplicationCommandPermissionData {
  return {
    id: guild.ownerId,
    type: "USER",
    permission: true
  }
}
