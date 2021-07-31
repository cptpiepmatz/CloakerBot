import {Snowflake} from "discord.js";

import BaseGuildDBData from "./base/BaseGuildDBData";

/**
 * Interface for the database type for roles.
 */
export default interface RolesDBData extends BaseGuildDBData {
  takedownRole?: Snowflake;
}
