const {post} = require("./requests");
const pm = require('../test/pm');
const expect = require('chai').expect;
const orgMember = require("./orgMember");

const create = async function (saveVarName, eventName) {
    let requestBody = `{
    "name": "${eventName || "Mocha tests"}",
    "organization_id": "{{last_org_id}}",
    "venue_id": "{{last_venue_id}}",
    "event_start": "2020-11-13T12:00:00",
    "event_end": "2020-11-14T12:00:00",
    "event_type": "Music",
    "age_limit": "A custom age limit",
    "promo_image_url": "https://source.unsplash.com/random"
}`;

    await orgMember.login("__om_token");
    let response = await post('/events', requestBody, '{{__om_token}}');
    const responseBody = JSON.stringify(response.body);
    const json = JSON.parse(responseBody);
    expect(response.status).to.equal(201);
    pm.environment.set(saveVarName || "last_event_id", json.id);
    response =  await post( '/events/'+ json.id + '/publish', requestBody, '{{org_member_token}}');

    expect(response.status).to.equal(200);

    return json;
}


const createTickets = async function (event, saveVarName, quantity, price) {
    let requestBody = `{
	"name":"Default_Pricing_{{$timestamp}}",
	"capacity": ${quantity || 100},
	"start_date":"1982-02-01T02:22:00",
	"end_date": "9999-01-10T02:22:00",
	"price_in_cents": ${price == undefined ?  2500 : price},
	"limit_per_person": 0,
	"visibility": "Always",
	"ticket_pricing":[]
}`;
    const response = await post('/events/' + (event ? event.id : "{{last_event_id}}") + '/ticket_types'
        , requestBody, '{{org_member_token}}');
    const responseBody = JSON.stringify(response.body);
    const json = JSON.parse(responseBody);
    expect(response.status).to.equal(201);

    pm.environment.set(saveVarName || "last_ticket_type_id", JSON.parse(responseBody).id);


    return json;

}


module.exports = {
    create, createTickets
};
