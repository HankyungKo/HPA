use ark_bls12_381::Bls12_381;
use ark_dh_commitments::{
    afgho16::{AFGHOCommitmentG1, AFGHOCommitmentG2},
    identity::IdentityCommitment,
    // pedersen::PedersenCommitment,
    DoublyHomomorphicCommitment,
};
use ark_ec::{PairingEngine};
use ark_ff::{UniformRand};
use ark_inner_products::{
    ExtensionFieldElement, InnerProduct, PairingInnerProduct,
};
use ark_dory_with_zk::dory_with_zk::DORY;

use ark_std::rand::{rngs::StdRng, Rng, SeedableRng};
use blake2::Blake2b;
use digest::Digest;

use std::{ops::MulAssign, time::Instant}; //, env};




fn bench_dory<IP, LMC, RMC, IPC, P, D, R: Rng>(rng: &mut R, len: usize)
where
    D: Digest,
    P: PairingEngine,
    IP: InnerProduct<
        LeftMessage = LMC::Message,
        RightMessage = RMC::Message,
        Output = IPC::Message,
    >,
    LMC: DoublyHomomorphicCommitment<Scalar = P::Fr, Key = P::G2Projective, Message = P::G1Projective>,
    RMC: DoublyHomomorphicCommitment<Scalar = LMC::Scalar, Key = P::G1Projective, Message = P::G2Projective>,
    IPC: DoublyHomomorphicCommitment<Scalar = LMC::Scalar>,
    LMC::Message: MulAssign<P::Fr>,
    RMC::Message: MulAssign<P::Fr>,
    IPC::Message: MulAssign<P::Fr>,
    IPC::Key: MulAssign<P::Fr>,
    LMC::Output: MulAssign<P::Fr>,
    RMC::Output: MulAssign<P::Fr>,
    IPC::Output: MulAssign<P::Fr>,
    IPC::Output: MulAssign<LMC::Scalar>,
    IP::LeftMessage: UniformRand,
    IP::RightMessage: UniformRand,
    LMC::Output: MulAssign<LMC::Scalar>,
    // IPC::Message: AddAssign<LMC::Output>,
    // IPC::Message: AddAssign<RMC::Output>,
    // RMC::Output: AddAssign<LMC::Output>,
{
    let mut l = Vec::new(); 
    let mut r = Vec::new();

    for _ in 0..len {
        l.push(<IP::LeftMessage>::rand(rng));
        r.push(<IP::RightMessage>::rand(rng));
    }

    let (gamma2, gamma1) = DORY::<IP,LMC,RMC,IPC, D>::setup(rng, len).unwrap();
    
    let r_c = <LMC as DoublyHomomorphicCommitment>::Scalar::rand(rng);
    let r_d1 = <LMC as DoublyHomomorphicCommitment>::Scalar::rand(rng);
    let r_d2 = <LMC as DoublyHomomorphicCommitment>::Scalar::rand(rng);
    let mut h1 = Vec::new();
    let mut h2 = Vec::new();
    h1.push(<IP::LeftMessage>::rand(rng));
    h2.push(<IP::RightMessage>::rand(rng));

    let (c, d1, d2)
         = DORY::<IP,LMC,RMC,IPC, D>::init_commit(&l, &r, &gamma1, &gamma2, &r_c, &r_d1, &r_d2, &h1, &h2).unwrap();

    let mut dory_srs = DORY::<IP, LMC, RMC, IPC, D>::precompute((&(gamma1.clone()), &(gamma2.clone())), &h1, &h2).unwrap();

    let mut start = Instant::now();
    let mut proof =
        DORY::<IP, LMC, RMC, IPC, D>::prove((&(l.clone()), &(r.clone())),
         &dory_srs, 
         (&(gamma1.clone()), &(gamma2.clone())), 
        //  (&(d1.clone()), &(d2.clone()), &(c.clone())),
         (&r_c, &r_d1, &r_d2),
         rng
        ).unwrap();
    let mut bench = start.elapsed().as_millis();
    println!("\t proving time: {} ms", bench);
    start = Instant::now();
    let result = DORY::<IP, LMC, RMC, IPC, D>::verify(&mut dory_srs, (&(gamma1.clone()), &(gamma2.clone())),
         (&(d1.clone()), &(d2.clone()), &(c.clone())), &mut proof)
        .unwrap();
    bench = start.elapsed().as_millis();
    println!("\t verification time: {} ms", bench);
    println!("result : {}", result);
}


fn main() { 
    // let arg = env::args().nth(1).unwrap();
    // let len: usize =arg.parse().unwrap();

    const LEN: usize = 16;
    type GC1 = AFGHOCommitmentG1<Bls12_381>;
    type GC2 = AFGHOCommitmentG2<Bls12_381>;
    let mut rng = StdRng::seed_from_u64(0u64);

    println!("Benchmarking DORY_with_zk with vector length: {}", LEN);

    println!("1) Pairing inner product...");
    bench_dory::<
        PairingInnerProduct<Bls12_381>,
        GC1,
        GC2,
        IdentityCommitment<ExtensionFieldElement<Bls12_381>, <Bls12_381 as PairingEngine>::Fr>,
        Bls12_381,
        Blake2b,
        StdRng,
    >(&mut rng, LEN);

}
