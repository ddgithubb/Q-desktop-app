// NAT CHECKS NEED TO BE RECONSIDERED
// They aren't accurate as seen on firefox (but you can test it out)
// I think the problem is the multiple ice candidates

// Also consider trickling ice instead of gathering complete?
// Because ice candidates can persumably change throughout the connection?
    // I think since we are using stun, the performance shouldn't be as bad

// Instead there should be a robust mechanism to allow the connection to be tried,
// and then sync server has to manually validate these sdp requests to make sure that
// they can in fact be used to try to connect