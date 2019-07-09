const {post} = require("./requests");
const pm = require('../test/pm');const debug=require('debug'); let log = debug('bn-api');

const expect = require('chai').expect;


const login = async function(saveVarName) {
    const response = await post('/auth/token', `{
	"email":"{{last_org_member_email}}",
	"password": "itsasecret"
}`);
    expect(response.status).to.equal(200);
    const responseBody = JSON.stringify(response.body);
    const json = JSON.parse(responseBody);

    pm.environment.set(saveVarName || "org_member_token", json.access_token);

    return json.access_token;
}
module.exports = {
    login
};