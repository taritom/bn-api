const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events/{{last_event_id}}/holds';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
	"name":"Musician Tickets{{$timestamp}}",
	"hold_type":"Discount",
	"redemption_code" : "Yeaaaah{{$timestamp}}",
	"discount_in_cents" : 100,
	"end_at": "2019-01-01T12:00:00",
	"max_per_user" : 10,
	"ticket_type_id": "{{last_ticket_type_id}}",
	"quantity" : 30
	
}`;


describe('OrgMember - create hold', function () {
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

        pm.environment.set("last_hold_id", JSON.parse(responseBody).id);

    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    })


});

            
