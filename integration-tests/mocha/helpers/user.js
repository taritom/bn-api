const {post}= require("./requests");
const pm = require('../test/pm');



let requestBody = `{
	"first_name":"User",
	"last_name":"Surname",
	"email":"{{last_email}}",
	"phone":"555",
	"password": "itsasecret"
}`;

const registerAndLogin = async function(saveVarName) {
    pm.environment.set("last_email", "user" + (new Date()).getTime() + "@tari.com");
    const response = await post( '/users', requestBody);
    const responseBody = JSON.stringify(response.body);
    const json = JSON.parse(responseBody);

    pm.environment.set(saveVarName || "user_token", json.access_token);

}

module.exports = {
    registerAndLogin
};