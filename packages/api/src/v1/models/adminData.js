/**
 * @swagger
 * definitions:
 *   ADMIN_DATA:
 *     type: object
 *     properties:
 *       EMAIL:
 *         type: string
 *       TOKEN:
 *         type: string
 *       required:
 *         - EMAIL
 *         - TOKEN
 */

module.exports = (sequelize, Sequelize) =>
  sequelize.define("ADMIN_DATA", {
    EMAIL: {
      type: Sequelize.STRING,
      primaryKey: true,
    },
    TOKEN: {
      type: Sequelize.STRING,
      allowNull: false,
    }
  });
