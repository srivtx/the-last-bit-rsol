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

    fn swap(&mut self) -> Option<( u64, u64) > { 
         if self.balance_a > 0 && self.balance_b > 0 { 
            let a = self.balance_a ; 
            let b = self.balance_b ; 

            self.balance_a = 0 ; 
            self.balance_b = 0 ; 

            Some((a,b)) 
         }else { 
            None  
         }
    }

}


fn main (){
    let mut escrow   =  Escrow::new() ; 

    escrow.deposit_a(10);

    escrow.deposit_b(5 );
    println!("A balance: {}", escrow.balance_a);
    println!("B balance: {}", escrow.balance_b);



    match escrow.swap() { 
        Some((a, b))  => println!(" swapped with  a {} , b  {}" , a , b)  , 
        None => println!("both parties must deposit first")
    }

    println!("A balance: {}", escrow.balance_a);
    println!("B balance: {}", escrow.balance_b); 
}
