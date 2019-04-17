const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events/{{last_event_id}}/ticket_types';


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
	"name":"General Admission2_{{$timestamp}}",
	"capacity": 1000,
	"start_date":"1982-02-01T02:22:00",
	"end_date": "9999-01-10T02:22:00",
	"price_in_cents": 2500,
	"visibility": "Always",
	"limit_per_person": 50,
	"ticket_pricing":[{
		"name": "Standard",
		"price_in_cents": 3000,
		"start_date":"1982-02-01T02:22:00",
		"end_date": "2022-02-01T02:22:00"
	},
	{
		"name": "Test_{{$timestamp}}_box_office_only",
		"price_in_cents": 4000,
		"is_box_office_only": true,
		"start_date":"1982-02-01T02:22:00",
		"end_date": "9999-01-10T02:22:00"
	}
	]
}`;


describe('OrgMember - create tickets - GA for cancellation', function () {
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


        pm.environment.set("tickets_to_cancel", JSON.parse(responseBody).id);
    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    })


});

            
