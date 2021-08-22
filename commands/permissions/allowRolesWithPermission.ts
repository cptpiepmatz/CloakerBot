import {
  ApplicationCommandPermissionData,
  Guild,
  PermissionResolvable
} from "discord.js";

export default function allowRolesWithPermission(
  guild: Guild, permission: PermissionResolvable
): ApplicationCommandPermissionData[] {
  const permissionData: ApplicationCommandPermissionData[] = [];
  for (let role of guild.roles.cache.values()) {
    if (role.permissions.has(permission)) {
      permissionData.push({
        id: role.id,
        type: "ROLE",
        permission: true
      })
    }
  }
  return permissionData;
}
