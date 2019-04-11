const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/codes/{{last_code_id}}';


var response;
var responseBody;


const put = async function (request_body) {
    return baseUrl
        .put(pm.substitute(apiEndPoint))
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
			"name" : "New Discount Name",
			"discount_as_percentage": 5
}`;


describe('OrgMember - update discount', function () {
    before(async function () {
        response = await put(requestBody);
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


    });

    it("should be 200", function () {
        expect(response.status).to.equal(200);
    });


    it("discount should have correct information", function () {
        let json = JSON.parse(responseBody);
        expect(json.name).to.equal("New Discount Name");
        expect(json.max_uses).to.equal(10);
        expect(json.discount_in_cents).to.null;
        expect(json.discount_as_percentage).to.equal(5);
        expect(json.start_date).to.equal("2018-01-01T12:00:00");
        expect(json.end_date).to.equal("2059-01-01T12:00:00");
        expect(json.max_tickets_per_user).to.equal(10);
        let ticket_type_id = pm.variables.get("last_ticket_type_id");
        expect(json.ticket_type_ids[0]).to.equal(ticket_type_id);
    });


});

            
