import {ApplicationCommandPermissionData, Guild} from "discord.js";
import ownerId from "../../provider/ownerId";

export default function allowBotOwner(
  guild: Guild
): ApplicationCommandPermissionData {
  return {
    id: ownerId,
    type: "USER",
    permission: true
  }
}
