struct User {
    email: String,
    phone: Option<String>,
    name: Option<String>,
    age: Option<u8>,
    posts: Vec<Post>,
    groups: Vec<Group>,
}

struct Post {
    title: String,
    content: String,
    author: User,
}

struct Group {
    name: String,
    members: Vec<User>,
}
