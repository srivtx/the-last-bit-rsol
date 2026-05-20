struct Escrow { 
    balance_a : u64 , 
    balance_b : u64 , 
}

impl Escrow {
    
     
    fn new() -> Self { 
        Self { 
            balance_a : 0    ,
            balance_b : 0 
        }
    }


    fn deposit_a (&mut self , amount : u64 ){
        self.balance_a += amount ; 
    }
    

    fn deposit_b( &mut self , amount : u64 ){
        self.balance_b += amount ; 
    }



}


fn main (){
    let mut escrow   =  Escrow::new() ; 

    escrow.deposit_a(10);
    escrow.deposit_b(20);



    println!("A balance: {}", escrow.balance_a);
    println!("B balance: {}", escrow.balance_b);

}
