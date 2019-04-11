const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/users';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))


        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
	"first_name":"Box",
	"last_name":"Office",
	"email":"{{last_boxoffice_email}}",
	"phone":"555",
	"password": "itsasecret"
}`;


describe('OrgBoxOffice - Register and Login', function () {
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

        pm.environment.set("org_boxoffice_token", json.access_token);


    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    })
    it("should have token response", function () {
        let json = JSON.parse(responseBody);
        expect(json).to.have.property("access_token");
        expect(json).to.have.property("refresh_token");
    });


});

            
