shopt -s extglob
scp -r !(target) winvmadmin:"/c:/Users/Eva/time-guardian-full/"
ssh winvm 'cd c:\Users\Eva\time-guardian-full && cargo run -F deploy -- run'
