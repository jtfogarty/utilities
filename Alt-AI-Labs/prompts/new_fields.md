

I want to be able to execute `./x_bookmark_puller populate_new_fields` which updates the x_bookmarks 
table with the new fields.

using the bookmark_id of `1804016740822548855` and the below returns the actual tweet. 
```
let base = "https://api.x.com/2/tweets/1804016740822548855"; 
```
`of course we do not want to hard code the id`

The the tweet id is used for the bookmark id

so the process is to select all bookmark id from x_bookmarks

then for each bookmark get `note_tweet` and `article` fields

then update x_bookmarks fields `note_tweet` and `article` if there is data in them. 

We also do not want to violate how much we can pull in a given 15 minutes.  




