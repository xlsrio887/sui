module a::m {
    use sui::object::UID;
    public enum S has key {
        N { id: UID }
    }
}
