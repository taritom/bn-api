const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/tickets/transfer';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{user_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
	"ticket_ids":[
		"{{ticket1_id}}",
		"{{ticket2_id}}"]
}`;


describe('User - Tickets - Authorize Transfer', function () {
    before(async function () {
        response = await post(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);
    });

    after(async function () {
        // add after methods


        let json = JSON.parse(responseBody);

        pm.environment.set("transfer_transfer_key", json.transfer_key);
        pm.environment.set("transfer_sender_user_id", json.sender_user_id);
        pm.environment.set("transfer_num_tickets", json.num_tickets);
        pm.environment.set("transfer_signature", json.signature);


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    })


});
