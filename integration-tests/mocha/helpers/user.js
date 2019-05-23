const {post} = require("./requests");
const pm = require('../test/pm');const debug=require('debug'); let log = debug('bn-api');

const expect = require('chai').expect;


const registerAndLogin = async function (saveVarName) {
    let email  ="user" + (new Date()).getTime() + Math.floor((Math.random() * 1000000)) + "@tari.com";
    pm.environment.set("last_email", email);
    log(email);
    const response = await post('/users', `{
	"first_name":"User",
	"last_name":"Surname",
	"email":"${email}",
	"phone":"555",
	"password": "itsasecret"
}`);
    expect(response.status).to.equal(201);
    const responseBody = JSON.stringify(response.body);
    const json = JSON.parse(responseBody);

    pm.environment.set(saveVarName || "user_token", json.access_token);

    return json.access_token;
}
module.exports = {
    registerAndLogin
};
