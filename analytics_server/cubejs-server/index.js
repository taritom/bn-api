const CubejsServer = require('@cubejs-backend/server');

const server = new CubejsServer({
    queryTransformer: (query, { authInfo }) => {
        const user = authInfo.u;
        // Todo: require auth
        if (user && user.event_id) {
            query.filters.push({
                dimension: 'Events.id',
                operator: 'equals',
                values: [user.event_id]
            })
        }
        return query;
    }
});

server.listen().then(({ port }) => {
  console.log(`🚀 Cube.js server is listening on ${port}`);
}).catch(e => {
  console.error('Fatal error during server start: ');
  console.error(e.stack || e);
});
